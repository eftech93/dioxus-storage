// Initialize MongoDB with 100 sample products
db = db.getSiblingDB('products_db');

// Create products collection
db.createCollection('products');

// Generate 100 products
const categories = ['Electronics', 'Clothing', 'Food', 'Books', 'Home', 'Sports', 'Toys', 'Beauty'];
const brands = ['TechCorp', 'StyleInc', 'FreshFoods', 'BookWorld', 'HomePlus', 'SportMax', 'ToyJoy', 'BeautyBar'];
const colors = ['Red', 'Blue', 'Green', 'Black', 'White', 'Yellow', 'Purple', 'Orange'];

const now = Date.now();

const products = [];

for (let i = 1; i <= 100; i++) {
    const category = categories[i % categories.length];
    const brand = brands[i % brands.length];
    const color = colors[i % colors.length];
    
    products.push({
        id: `prod_${i.toString().padStart(4, '0')}`,
        name: `${brand} ${category} Item ${i}`,
        description: `High quality ${category.toLowerCase()} product from ${brand}. Available in ${color} color.`,
        price: parseFloat((Math.random() * 500 + 10).toFixed(2)),
        category: category,
        brand: brand,
        color: color,
        stock: Math.floor(Math.random() * 100),
        rating: parseFloat((Math.random() * 5).toFixed(1)),
        in_stock: Math.random() > 0.2,
        created_at: now - Math.floor(Math.random() * 365 * 24 * 60 * 60 * 1000),
        updated_at: now
    });
}

// Insert products
db.products.insertMany(products);

// Create indexes for better query performance
db.products.createIndex({ id: 1 }, { unique: true });
db.products.createIndex({ category: 1 });
db.products.createIndex({ brand: 1 });
db.products.createIndex({ price: 1 });

print(`Inserted ${products.length} products into the database`);
